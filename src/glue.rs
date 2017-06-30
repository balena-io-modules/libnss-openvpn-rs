use {NssStatus, gethostbyname};

use libc::{c_char, size_t, hostent, AF_INET};
use libc::{ENOENT, ERANGE};
use std::ffi::CStr;
use std::str;
use std::mem::size_of;

#[no_mangle]
pub extern "C" fn _nss_openvpn_gethostbyname2_r(
    name: *const c_char,
    af: i32,
    result: *mut hostent,
    buffer: *mut c_char,
    buflen: size_t,
    errnop: *mut i32,
    h_errnop: *mut i32,
) -> NssStatus {
    if af != AF_INET {
        unsafe { *errnop = ENOENT; }
        return NssStatus::NotFound;
    }

    _nss_openvpn_gethostbyname_r(name, result, buffer, buflen, errnop, h_errnop)
}

struct HostData {
    name: [u8; 256],
    addr_list: [*mut u32; 2],
    s_addr: u32
}

#[no_mangle]
pub extern "C" fn _nss_openvpn_gethostbyname_r(
    name: *const c_char,
    result: *mut hostent,
    buffer: *mut c_char,
    buflen: size_t,
    errnop: *mut i32,
    h_errnop: *mut i32,
) -> NssStatus {
    assert!(!result.is_null() && !name.is_null() && !buffer.is_null());
    unsafe { *h_errnop = 0; }

    if buflen < size_of::<HostData>() {
        unsafe { *errnop = ERANGE };
        return NssStatus::TryAgain;
    }

    let name = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s,
        Err(_) => {
            unsafe { *errnop = ENOENT };
            return NssStatus::NotFound;
        }
    };

    match gethostbyname(name) {
        Ok(ip) => {
            unsafe {
                let mut data = buffer as *mut HostData;
                (*result).h_addrtype = AF_INET;
                (*result).h_length = size_of::<u32>() as i32;
                (*result).h_aliases = 0 as *mut *mut i8;

                (*result).h_addr_list = (*data).addr_list.as_mut_ptr() as *mut *mut i8;
                (*data).name[..name.len()].copy_from_slice(name.as_bytes());
                (*data).name[name.len()] = 0;
                (*data).s_addr = u32::from(ip).to_be();
                (*data).addr_list[0] = &mut (*data).s_addr;
                (*data).addr_list[1] = 0 as *mut u32;
            }

            return NssStatus::Success;
        },
        Err(code) => {
            unsafe { *errnop = ENOENT };
            return code;
        }
    }
}
