#include <stdio.h>
#include <string.h>
#include <linux/if.h>
#include <linux/if_tun.h>
#include <sys/ioctl.h>
#include <fcntl.h>


int getfd() {
    int fd;
    struct ifreq ifr;

    fd = open("/dev/net/tun", O_RDWR);

    if (fd < 0) {
        printf("failed to open /dev/net/tun\n");
        return -1;
    }

    memset(&ifr, 0, sizeof(ifr));
    ifr.ifr_flags = IFF_TUN | IFF_NO_PI;
    strncpy(ifr.ifr_name, "artes", IFNAMSIZ);

    if (ioctl(fd, TUNSETIFF, &ifr) < 0) {
        printf("ioctl TUNSETIFF");
        return -1;
    }

    return fd;
}