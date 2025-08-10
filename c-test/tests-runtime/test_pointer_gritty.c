void putchar(int c);

void touch(int *q){ *q = 7; }

int main() {
  int z = 0; touch(&z);
  putchar(z+'0');
}