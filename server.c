#include <stdio.h>
#include <netdb.h>
#include <string.h>
#include <sys/types.h>
#include <sys/socket.h>

#define SERVER_PORT 4242

int main() {
    char buf[100];
    struct sockaddr_in servaddr;
    int server_fd, comm_fd;

    server_fd = socket(AF_INET, SOCK_STREAM, 0);

    bzero(&servaddr, sizeof(servaddr));

    servaddr.sin_family  = AF_INET;
    servaddr.sin_addr.s_addr  = htons(INADDR_ANY);
    servaddr.sin_port  = htons(SERVER_PORT);

    bind(server_fd, (struct sockaddr *) &servaddr, sizeof(servaddr));

    listen(server_fd, 10);

    comm_fd = accept(server_fd, (struct sockaddr*) NULL, NULL);

    while(1)
    {
        bzero(buf, 100);
        read(comm_fd, buf, 100);
        write(comm_fd, buf, strlen(buf)+1);
    }
}
