void putchar(int c);

void touch(int *q){ *q = 7; }


int main() {
//  int a[6] = {0,1,2,3,4,5};

//  int arr[6];
//  arr[0] = 0;
//  arr[1] = 1;
//  arr[2] = 2;
//  arr[3] = 3;
//  arr[4] = 4;
//  arr[5] = 5;
//
//  int *p = arr;
//  *(p+3) = 9;               // a[3] = 9
//  putchar(arr[3]+'0');
//
  int z = 0; touch(&z);
  putchar(z+'0');
}