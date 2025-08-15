struct Inner {
    int a;
    int b;
};
struct Outer {
    int x;
    struct Inner inner;
    int y;
};
int main() {
    struct Outer obj;
    return sizeof(struct Outer);
}
