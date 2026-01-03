#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#include "chameleon.h"

#define OUTPUT_LENGTH (4*4096)

/*
Generator: 84 705416858 ~126 MiB/s
Mutator: 69 531055780 ~155 MiB/s
*/

struct timespec diff_timespec(struct timespec *time1,
    struct timespec *time0) {
  struct timespec diff = {.tv_sec = time1->tv_sec - time0->tv_sec, //
      .tv_nsec = time1->tv_nsec - time0->tv_nsec};
  if (diff.tv_nsec < 0) {
    diff.tv_nsec += 1000000000; // nsec/sec
    diff.tv_sec--;
  }
  return diff;
}

int main (void) {
    ChameleonWalk walk;
    unsigned char* output = malloc(OUTPUT_LENGTH);
    size_t total = 0;
    struct timespec start, end;
    
    chameleon_init(walk, 4096);
    chameleon_seed(time(NULL));
    
    clock_gettime(CLOCK_MONOTONIC, &start);
    while (total < 10UL * 1024 * 1024 * 1024) {
        total += chameleon_mutate(walk, output, OUTPUT_LENGTH);
    }
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    struct timespec diff = diff_timespec(&end, &start);
    printf("%lu %lu\n", diff.tv_sec, diff.tv_nsec);
    
    chameleon_destroy(walk);
    free(output);
}
