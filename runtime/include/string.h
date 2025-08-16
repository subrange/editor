#ifndef STRING_H
#define STRING_H

// Standard string/memory functions

// Copy n bytes from src to dest
// Note: Current implementation uses fat pointers internally
// but the interface is standard C
void memcpy(void *dest, const void *src, int n);

// Set n bytes of memory to value c
void memset(void *s, int c, int n);

// TODO: Future additions
// int strlen(const char *s);
// char *strcpy(char *dest, const char *src);
// char *strncpy(char *dest, const char *src, int n);
// int strcmp(const char *s1, const char *s2);
// int strncmp(const char *s1, const char *s2, int n);
// char *strcat(char *dest, const char *src);
// char *strchr(const char *s, int c);
// char *strstr(const char *haystack, const char *needle);

#endif // STRING_H