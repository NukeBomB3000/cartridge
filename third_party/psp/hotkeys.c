/* A keyboard that is not there, for PPSSPP to be told things on: hotkeys < FIFO. PPSSPP only
 * takes its hotkeys (save a state, leave, the pause menu) from a keyboard, and the handheld has
 * none; so a virtual one is made, with the four keys PPSSPP is set to take them on, and a word
 * on stdin types the key: "save" is F2, "load" F4, "exit" F10 and "pause" Escape (see
 * System/psp.sh, which writes down the keys in PPSSPP's settings). It makes the keyboard
 * before PPSSPP starts, as SDL looks for the devices there are only once. */
#include <fcntl.h>
#include <linux/uinput.h>
#include <stdio.h>
#include <string.h>
#include <sys/ioctl.h>
#include <unistd.h>

static int fd;

static void emit(int type, int code, int value)
{
    struct input_event e;
    memset(&e, 0, sizeof(e));
    e.type = type;
    e.code = code;
    e.value = value;
    if (write(fd, &e, sizeof(e)) < 0) {
        perror("hotkeys: write");
    }
}

static void tap(int key)
{
    emit(EV_KEY, key, 1);
    emit(EV_SYN, SYN_REPORT, 0);
    usleep(80000);
    emit(EV_KEY, key, 0);
    emit(EV_SYN, SYN_REPORT, 0);
}

int main(void)
{
    char line[64];
    struct uinput_setup setup;
    const int keys[] = { KEY_F2, KEY_F4, KEY_F10, KEY_ESC };

    fd = open("/dev/uinput", O_WRONLY | O_NONBLOCK);
    if (fd < 0) {
        perror("hotkeys: /dev/uinput");
        return 1;
    }
    ioctl(fd, UI_SET_EVBIT, EV_KEY);
    for (unsigned i = 0; i < sizeof(keys) / sizeof(keys[0]); i++) {
        ioctl(fd, UI_SET_KEYBIT, keys[i]);
    }
    memset(&setup, 0, sizeof(setup));
    setup.id.bustype = BUS_VIRTUAL;
    setup.id.vendor = 0x1209;
    setup.id.product = 0xca47;
    snprintf(setup.name, sizeof(setup.name), "cartridge-hotkeys");
    ioctl(fd, UI_DEV_SETUP, &setup);
    ioctl(fd, UI_DEV_CREATE);
    /* the node needs a moment to appear under /dev/input */
    usleep(500000);
    puts("ready");
    fflush(stdout);

    while (fgets(line, sizeof(line), stdin)) {
        if (strncmp(line, "save", 4) == 0) {
            tap(KEY_F2);
        } else if (strncmp(line, "load", 4) == 0) {
            tap(KEY_F4);
        } else if (strncmp(line, "exit", 4) == 0) {
            tap(KEY_F10);
        } else if (strncmp(line, "pause", 5) == 0) {
            tap(KEY_ESC);
        }
    }
    ioctl(fd, UI_DEV_DESTROY);
    close(fd);
    return 0;
}
