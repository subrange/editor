void putchar(int c);

void increment_all(int *arr, int n) {
    int i = 0;
    while (i < n) {
        arr[i] = arr[i] + 1;
        i = i + 1;
    }
}

int main() {
    int nums[3];
    nums[0] = 5; nums[1] = 6; nums[2] = 7;
    increment_all(nums, 3);
    
    // Should be 6, 7, 8 after increment
    // Sum should be 21
    int sum = nums[0] + nums[1] + nums[2];
    
    if (sum == 21) {
        putchar('Y');
    } else {
        putchar('0' + (sum - 20));  // Show the difference
    }
    putchar('\n');
    
    return 0;
}
