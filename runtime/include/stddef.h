#ifndef STDDEF_H
#define STDDEF_H

// Standard definitions

// Null pointer constant
#ifndef NULL
#define NULL ((void *)0)
#endif

// Size type (using int for now, should be unsigned)
typedef int size_t;

// Pointer difference type
typedef int ptrdiff_t;

// Offset of member in structure
#define offsetof(type, member) ((size_t) &((type *)0)->member)

#endif // STDDEF_H