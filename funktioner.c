#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <windows.h>
static char* _ms_strjoin(const char* a, const char* b) {
    size_t la = strlen(a), lb = strlen(b);
    char* r = (char*)malloc(la + lb + 1);
    memcpy(r, a, la); memcpy(r + la, b, lb + 1); return r;
}
void hilsen(char* navn) {
    printf("%s\n", _ms_strjoin(_ms_strjoin("Hej, ", navn), "!"));
}
int main(void) {
    SetConsoleOutputCP(65001);
    hilsen("verden");
    hilsen("Danmark");
    return 0;
}