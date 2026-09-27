use std::process::Command;
use std::io;
use crate::utils::{ sh };

pub fn create_tunnel(name: &str, ip: &str) -> io::Result<()> {
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