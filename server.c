#include <stdio.h>
#include <stdlib.h>
#include <netdb.h>
#include <string.h>
#include <sys/types.h>
#include <sys/socket.h>

#define SERVER_PORT 4242
#define BUF_SIZE 256

int accept_next(int fd) {
    int comm_fd = accept(fd, (struct sockaddr*) NULL, NULL);
    if (comm_fd < 0)
        error("ERROR: Accepting");
    return comm_fd;
}

int read_next(int fd, char *buf) {
    int i, n;
    char exp[4];
    uint32_t num_expected;

    bzero(exp, 4);
    n = read(fd, exp, 4);
    if (n < 0)
        error("ERROR: Can't read number of expected bytes.");
    else if (n == 0)
        return 0;

    num_expected = (exp[0] << 24) | (exp[1] << 16) | (exp[2] << 8) | exp[3];
    num_expected = num_expected > BUF_SIZE ? BUF_SIZE : num_expected; 
    printf("Expecting %d\n", num_expected);
    
    n = read(fd, buf, num_expected);
    if (n < 0)
        error("ERROR: Can't read data.");
    printf("DATA: %s\n", buf);

    return n;
}

int main() {
    struct sockaddr_in servaddr;
    int n, listen_fd, comm_fd;
    char buf[BUF_SIZE];
    char output[BUF_SIZE];

    listen_fd = socket(AF_INET, SOCK_STREAM, 0);

    bzero(&servaddr, sizeof(servaddr));

    servaddr.sin_family  = AF_INET;
    servaddr.sin_addr.s_addr  = htons(INADDR_ANY);
    servaddr.sin_port  = htons(SERVER_PORT);

    bind(listen_fd, (struct sockaddr *) &servaddr, sizeof(servaddr));

    listen(listen_fd, 10);

    comm_fd = accept_next(listen_fd);
    while(1)
    {
        bzero(buf, BUF_SIZE);
        bzero(output, BUF_SIZE);
        n = read_next(comm_fd, buf);
        if (n > 0) {
            sprintf(output, "%s\n", buf);
            n = write(comm_fd, output, strlen(buf)+1);
            if (n < 0)
                error("ERROR: Can't write");
        } else if(n == 0) {
            close(comm_fd);
            comm_fd = accept_next(listen_fd);
        }
    }
}
