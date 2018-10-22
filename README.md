libnss-openvpn
==============

The libnss-openvpn name service switch module resolves the name
“xxx.balena” to the associated xxx machine in the openvpn.server.status
file.

With this module, you can access to a given host from the openvpn server
even if the IP address is not static.

```
    $ ssh foobar.balena
```

Configuration
-------------

The module uses openvpn status files to get the host information. It globs over
the following pattern `/var/run/openvpn/server-*.status`. This allows you to
run multiple OpenVPN servers, each with its own status file.

If the file is missing you can add the following line in your `server.conf` file:

```
status /var/run/openvpn/server-0.status
```

If you want users to be able to use the module, you will probably
need to relax the security of `server.status`:

```
chmod 644 /var/run/openvpn/server-0.status
```

### NSS Configuration

Add the `openvpn` service to the `/etc/nsswitch.conf` file:

```
hosts:         compat openvpn
```

Installation
------------

Upon building the library using `cargo build --release`, copy the
`target/release/libnss_openvpn.so` file to
`/lib/???-linux-gnu/libnss_openvpn.so.2`.

Credits
-------

This library is based on https://github.com/goneri/libnss-openvpn
