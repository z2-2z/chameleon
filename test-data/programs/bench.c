#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#include "chameleon.h"

#define OUTPUT_LENGTH (4 * 4096)

/*
Generator: 86 105185398 invalid=(18/154105364) ~125 MiB/s or 1.79m gens/s
Mutator: 71 634873447 invalid=(245/9858496) ~150 MiB/s or 137k muts/s
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
    size_t total = 0, r, tries = 0, invalid = 0;
    struct timespec start, end;
    
    chameleon_init(walk, 4 * 4096);
    chameleon_seed(time(NULL));
    
    clock_gettime(CLOCK_MONOTONIC, &start);
    while (total < 10UL * 1024 * 1024 * 1024) {
        //r = chameleon_generate(walk, output, OUTPUT_LENGTH);
        r = chameleon_mutate(walk, output, OUTPUT_LENGTH);
        tries++;
        invalid += (r == OUTPUT_LENGTH);
        total += r;
    }
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    struct timespec diff = diff_timespec(&end, &start);
    printf("%lu %lu invalid=(%lu/%lu)\n", diff.tv_sec, diff.tv_nsec, invalid, tries);
    
    chameleon_destroy(walk);
    free(output);
}
