// Nested compound literal tests
void putchar(int c);

typedef struct {
    int x;
    int y;
} Point;

typedef struct {
    Point p1;
    Point p2;
} Line;

typedef struct {
    int data[3];
} Array3;

int main() {
    // Test 1: Nested struct with compound literals
    Line line = (Line){
        .p1 = (Point){1, 2},
        .p2 = (Point){3, 4}
    };
    if (line.p1.x == 1 && line.p1.y == 2 && 
        line.p2.x == 3 && line.p2.y == 4) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 2: Array of structs using compound literal
    Point *points = (Point[]){
        {5, 6},
        {7, 8},
        {9, 10}
    };
    if (points[0].x == 5 && points[0].y == 6 &&
        points[1].x == 7 && points[1].y == 8 &&
        points[2].x == 9 && points[2].y == 10) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 3: Struct containing array with compound literal
    Array3 a = (Array3){.data = {11, 12, 13}};
    if (a.data[0] == 11 && a.data[1] == 12 && a.data[2] == 13) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 4: 2D array compound literal
    int (*matrix)[2] = (int[][2]){{1, 2}, {3, 4}};
    if (matrix[0][0] == 1 && matrix[0][1] == 2 &&
        matrix[1][0] == 3 && matrix[1][1] == 4) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}