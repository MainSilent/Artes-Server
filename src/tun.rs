use std::process::Command;
use std::{io, os::fd::RawFd, io::Write};
use std::fs::File;
use std::os::fd::FromRawFd;
use crate::utils::{ sh };


pub fn tun_write(tun_fd: RawFd, buf: &[u8]) -> io::Result<()> {
    let mut tun = unsafe { File::from_raw_fd(tun_fd) };

    tun.write_all(buf)?;

    std::mem::forget(tun);

    Ok(())
}


pub fn get_tunnel_fd(name: &str) -> io::Result<RawFd> {
    let fd = unsafe { libc::open(b"/dev/net/tun\0".as_ptr() as _, libc::O_RDWR) };

    if fd < 0 {
        return Err(io::Error::last_os_error());
    }

    let mut ifr = unsafe { std::mem::zeroed::<libc::ifreq>() };

    for (dst, src) in ifr.ifr_name.iter_mut().zip(name.bytes()) {
        *dst = src as i8;
    }

    unsafe {
        ifr.ifr_ifru.ifru_flags = (libc::IFF_TUN | libc::IFF_NO_PI) as i16;

        if libc::ioctl(fd, libc::TUNSETIFF as _, &mut ifr) < 0 {
            let e = io::Error::last_os_error();
            libc::close(fd);
            return Err(e);
        }
    }

    Ok(fd)
}


pub fn create_tunnel_interface(name: &str, ip: &str) -> io::Result<()> {
    let nic = Command::new("sh")
        .arg("-c")
        .arg("ip -4 route | grep default | grep -Po '(?<=dev )\\S+' | head -1")
        .output()?;

    let dns = Command::new("sh")
        .arg("-c")
        .arg("ip -4 route | grep default | grep -Po '(?<=via )\\S+' | head -1")
        .output()?;

    let nic = String::from_utf8_lossy(&nic.stdout).trim().to_string();
    let dns = String::from_utf8_lossy(&dns.stdout).trim().to_string();

    sh(&format!(r#"
        if ip link show {name} >/dev/null 2>&1; then
            exit 0
        fi

        ip tuntap del {name} mode tun 2>/dev/null || true
        ip tuntap add {name} mode tun
        ip link set {name} up
        ip addr add {ip}/16 dev {name}

        echo 1 > /proc/sys/net/ipv4/ip_forward

        iptables -A FORWARD -i {name} -o {nic} -j ACCEPT
        iptables -A FORWARD -i {nic} -o {name} -m state --state ESTABLISHED,RELATED -j ACCEPT
        iptables -t nat -A POSTROUTING -o {nic} -j MASQUERADE

        iptables -t nat -A PREROUTING -p tcp --dport 53 -j DNAT --to-destination {dns} -i {name}
        iptables -t nat -A PREROUTING -p udp --dport 53 -j DNAT --to-destination {dns} -i {name}
    "#))?;

    Ok(())
}