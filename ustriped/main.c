#include <assert.h>
#include <stdlib.h>
#include <stdio.h>
#include <sys/time.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <netinet/in.h>
#include <string.h>

#define SPIDEV "/dev/spidev0.0"
#define LEDCOUNT 226

#define PKT_MAXLEN 65540
/* Idle time before switching to lower prio (= higher channel) */
#define PRIO_TIMEOUT 100000

typedef unsigned char byte;

#define CMD_SET_PIXEL_COLORS 0

signed long long now_usec() {
struct timeval tv;
assert(gettimeofday(&tv, NULL) == 0);
  return tv.tv_sec * 1000000 + tv.tv_usec;
}

int create_udp_server() {
  int udp_server = socket(AF_INET6, SOCK_DGRAM, 0);
  assert(udp_server >= 0);

  struct sockaddr_in6 listen_addr;
  memset(&listen_addr, 0, sizeof(listen_addr));
  listen_addr.sin6_family = AF_INET6;
  listen_addr.sin6_port = htons(2342);
  assert(bind(udp_server, (struct sockaddr *)&listen_addr, sizeof(listen_addr)) == 0);

  return udp_server;
}

int spifd;

void set_pixel_colours(const byte *data, ssize_t data_len) {
  static signed long long last_write = 0;
  signed long long to_wait = last_write + 1000 - now_usec();
  if (last_write != 0 && to_wait > 0) {
    printf("last_write = %lli us\n", last_write);
    printf("now = %lli us\n", now_usec());
    printf("to_wait: %lli us\n", to_wait);
    usleep(to_wait);
  }

  if (data_len != 3 * LEDCOUNT) {
    fprintf(stderr, "data_len = %lu != %u\n", data_len, 3 * LEDCOUNT);
    return;
  }

  assert(write(spifd, (void *)data, data_len) == data_len);
}

void handle_message(byte channel, byte command, byte *data, ssize_t data_len) {
  static byte current_priority = 255;
  static long long last_message = 0;
  long long now = now_usec();

  if (channel < current_priority || last_message + PRIO_TIMEOUT <= now) {
    current_priority = channel;
    last_message = now;
  } else if (channel > current_priority) {
    return;
  }

  switch(command) {
  case CMD_SET_PIXEL_COLORS:
    set_pixel_colours(data, data_len);
    break;
  }

  last_message = now;
}

int main() {
  int udp_server = create_udp_server();
  spifd = open(SPIDEV, O_WRONLY);
  assert(spifd >= 0);

  while(1) {
    byte buf[PKT_MAXLEN];
    ssize_t buf_len = recvfrom(udp_server, buf, PKT_MAXLEN, 0, NULL, 0);
    assert(buf_len >= 0);
    if (buf_len < 4) {
      fprintf(stderr, "Short packet: %lu\n", buf_len);
      continue;
    }

    byte channel = buf[0];
    byte command = buf[1];
    unsigned short data_len = (buf[2] << 8) | buf[3];
    if (buf_len != 4 + data_len) {
      fprintf(stderr, "Incorrect pkt length %lu != %u\n", buf_len, 4 + data_len);
      continue;
    }

    handle_message(channel, command, buf + 4, data_len);
  }
}
