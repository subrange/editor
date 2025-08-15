unsigned short __ld8(unsigned short bank, unsigned short byte_addr);
void __st8(unsigned short bank, unsigned short byte_addr, unsigned short value);
void putchar(int c);

int main() {
  __asm__("LI X0, 0x4241;");
  __asm__("LI X1, 0x1;");
  __asm__("STORE X0, X1, X1;");
  int a = __ld8(1, 2);
  putchar(a);
  a = __ld8(1, 3);
  putchar(a); // AB

  putchar('\n');

  __st8(1, 3, 0x43);

  int b = __ld8(1, 3);
  putchar(b); // C
}