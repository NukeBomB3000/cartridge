/* Prints the SDL game controller mapping for the handheld's own buttons, for PPSSPP's SDL to take
 * the buttons as a pad's: padmap [/dev/input/js0]. SDL has no mapping for the handheld's keys,
 * which its drivers name as they like, and the buttons' numbers differ from one handheld to the
 * next with the keys it has (a stick adds some), so the numbers are read from the driver: the
 * code of the key each joystick button stands for, and the code of each axis. The roles are
 * the frontend's (input/evdev.rs, the handheld layout): A is the right face button, B the lower,
 * X the upper and Y the left one; the Nintendo way round, which is the PSP's by position. */
#include <fcntl.h>
#include <linux/joystick.h>
#include <stdio.h>
#include <sys/ioctl.h>
#include <string.h>
#include <unistd.h>

#include "SDL.h"

#define BTN_SOUTH_A 0x130 /* the handheld's A */

static int button(const unsigned short *codes, int n, int code)
{
    for (int i = 0; i < n; i++) {
        if (codes[i] == code) {
            return i;
        }
    }
    return -1;
}

static int axis(const unsigned char *codes, int n, int code)
{
    for (int i = 0; i < n; i++) {
        if (codes[i] == code) {
            return i;
        }
    }
    return -1;
}

int main(int argc, char **argv)
{
    const char *path = argc > 1 ? argv[1] : "/dev/input/js0";
    unsigned char buttons = 0, axes = 0;
    unsigned short btnmap[KEY_MAX - BTN_MISC + 1];
    unsigned char axmap[ABS_CNT];
    char name[128] = "";
    char guid[64];
    int fd = open(path, O_RDONLY);
    if (fd < 0) {
        perror(path);
        return 1;
    }
    ioctl(fd, JSIOCGNAME(sizeof(name)), name);
    ioctl(fd, JSIOCGBUTTONS, &buttons);
    ioctl(fd, JSIOCGAXES, &axes);
    memset(btnmap, 0, sizeof(btnmap));
    memset(axmap, 0, sizeof(axmap));
    ioctl(fd, JSIOCGBTNMAP, btnmap);
    ioctl(fd, JSIOCGAXMAP, axmap);
    close(fd);

    SDL_setenv("SDL_JOYSTICK_DEVICE", path, 1);
    if (SDL_Init(SDL_INIT_JOYSTICK) != 0 || SDL_NumJoysticks() < 1) {
        fprintf(stderr, "padmap: SDL sees no joystick at %s\n", path);
        return 1;
    }
    SDL_JoystickGetGUIDString(SDL_JoystickGetDeviceGUID(0), guid, sizeof(guid));

    printf("%s,%s,platform:Linux", guid, name);
    const struct {
        const char *role;
        int code;
    } roles[] = {
        { "a", 0x131 },             /* the handheld's B: the lower face button */
        { "b", BTN_SOUTH_A },       /* A: the right one */
        { "x", 0x132 },             /* Y: the left one */
        { "y", 0x133 },             /* X: the upper one */
        { "leftshoulder", 0x134 },  /* L1 */
        { "rightshoulder", 0x135 }, /* R1 */
        { "back", 0x136 },          /* SELECT */
        { "start", 0x137 },
        { "guide", 0x138 },         /* MENU */
        { "lefttrigger", 0x13a },   /* L2 */
        { "righttrigger", 0x13b },  /* R2 */
        { "leftstick", 0x139 },     /* L3 */
        { "rightstick", 0x13c },    /* R3 */
    };
    for (size_t i = 0; i < sizeof(roles) / sizeof(roles[0]); i++) {
        int b = button(btnmap, buttons, roles[i].code);
        if (b >= 0) {
            printf(",%s:b%d", roles[i].role, b);
        }
    }
    printf(",dpup:h0.1,dpright:h0.2,dpdown:h0.4,dpleft:h0.8");
    /* the sticks of a handheld that has them arrive on ABS_Z, ABS_RX, ABS_RY and ABS_RZ: the
     * left one on the first two, the right on the others */
    int lx = axis(axmap, axes, 0x02), ly = axis(axmap, axes, 0x03);
    int rx = axis(axmap, axes, 0x04), ry = axis(axmap, axes, 0x05);
    if (lx >= 0 && ly >= 0 && rx >= 0 && ry >= 0) {
        printf(",leftx:a%d,lefty:a%d,rightx:a%d,righty:a%d", lx, ly, rx, ry);
    }
    printf(",\n");
    return 0;
}
