// Test that pointer arithmetic generates GEP instructions
int main() {
    int arr[10];
    int *p = arr;
    int *q = p + 5;  // Should generate GEP
    return *q;
}