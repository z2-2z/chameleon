#ifndef _PEACOCK_H
#define _PEACOCK_H

typedef void* PeacockWalk;

void peacock_init (PeacockWalk* walk, unsigned long capacity);
size_t peacock_mutate (PeacockWalk* walk, unsigned char* output, unsigned long output_length);
size_t peacock_generate (PeacockWalk* walk, unsigned char* output, unsigned long output_length);
int  peacock_parse(PeacockWalk* walk, unsigned char* input, unsigned long input_length);

#endif /* _PEACOCK_H */
