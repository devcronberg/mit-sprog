#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <windows.h>
static char* _ms_strjoin(const char* a, const char* b) {
    size_t la = strlen(a), lb = strlen(b);
    char* r = (char*)malloc(la + lb + 1);
    memcpy(r, a, la); memcpy(r + la, b, lb + 1); return r;
}
double forøg(double n) {
    return (n + 1);
}
int main(void) {
    SetConsoleOutputCP(65001);
    double x = forøg(5);
    printf("%g\n", x);
    printf("%g\n", forøg(forøg(10)));
    return 0;
}