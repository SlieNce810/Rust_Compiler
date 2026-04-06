func int main() {
    int a;
    int b;
    int c;
    a = 4;
    b = 3;
    c = a + b * 2;
    if (c > 5) {
        c = c - 1;
    } else {
        c = c + 1;
    }
    while (c > 0) {
        c = c - 1;
    }
    return c;
}
