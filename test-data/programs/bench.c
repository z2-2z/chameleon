#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#include "chameleon.h"

#define OUTPUT_LENGTH (4 * 4096)

/*
Generator: 89 970015720 invalid=(11/154148654) ~119 MiB/s or 1.7m gens/s
Mutator: 75 424618227 invalid=(249/9847650) ~142 MiB/s or 130k muts/s
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
        r = chameleon_generate(walk, output, OUTPUT_LENGTH);
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
