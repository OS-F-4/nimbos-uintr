/*
 * Build with

gcc cyclictest.c -lpthread

 * or

musl-gcc cyclictest.c -lpthread -DUSE_MUSL

 *
 * NOTE THAT FOR ACCURATE RESULTS    /dev/cpu_dma_latency    NEEDS TO BE SET TO 0.
 * See Documentation/power/pm_qos_interface.txt .
 *
 * DEFINES: USE_MUSL    MINIMAL
 */

#include <assert.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>

/*
 * number of timerthreads
 */
#define NUM_THREADS      1
#define MAX_CPUS         12
#define DEFAULT_INTERVAL 1000 // in usecs
#define DEFAULT_DISTANCE 500
// #define DEFAULT_PRIORITY 0
// #define DEFAULT_POLICY SCHED_OTHER // SCHED_FIFO
#define USEC_PER_SEC  1000000
#define NSEC_PER_SEC  1000000000
#define DEFAULT_CLOCK CLOCK_MONOTONIC
#define MAX_CYCLES    5000

struct thread_param {
    int id;
    pthread_t thread;
    unsigned long interval;
    int prio;
    int policy;
    int cpu; // which cpu to run on
};

struct thread_stat {
    int tid;
    long max;
    long min;
    long act;
    long sum; // not using `double avg`
    int cycles;
};

static int interval = DEFAULT_INTERVAL;
// static int priority = DEFAULT_PRIORITY;
static struct thread_param thrpar[NUM_THREADS];
static struct thread_stat thrstat[NUM_THREADS];
static int shutdown = 0;

static inline void tsnorm(struct timespec* ts)
{
    while (ts->tv_nsec >= NSEC_PER_SEC) {
        ts->tv_nsec -= NSEC_PER_SEC;
        ts->tv_sec++;
    }
}

static inline void tsinc(struct timespec* dst, const struct timespec* delta)
{
    dst->tv_sec += delta->tv_sec;
    dst->tv_nsec += delta->tv_nsec;
    tsnorm(dst);
}

// delta in usecs
static inline long tsdelta(const struct timespec* t1, const struct timespec* t2)
{
    int64_t diff = (long)USEC_PER_SEC * ((long)t1->tv_sec - (long)t2->tv_sec);
    diff += ((long)t1->tv_nsec - (long)t2->tv_nsec) / 1000;
    return diff;
}

static inline int tsgreater(struct timespec* a, struct timespec* b)
{
    return ((a->tv_sec > b->tv_sec) || (a->tv_sec == b->tv_sec && a->tv_nsec > b->tv_nsec));
}

static int timerthread(void* param)
{
    struct thread_param* par = param;
    struct thread_stat* stat = &thrstat[par->id];

    stat->tid = getpid();

    int interval = par->interval / 1000;

    while (!shutdown) {
        int time1 = get_time_ms();
        sleep(interval);
        int time2 = get_time_ms();
        // err = clock_nanosleep(DEFAULT_CLOCK, TIMER_ABSTIME, &next, NULL);
        // if (err == EINTR)
        //  break;
        // assert(!err && "cannot clock_nanosleep");

        // err = clock_gettime(DEFAULT_CLOCK, &now);
        // assert(!err);

        int diff = time2 - time1;

        // long diff = tsdelta(&now, &next);

        if (diff < stat->min)
            stat->min = diff;
        if (diff > stat->max)
            stat->max = diff;
        stat->act = diff;
        stat->sum += diff;
        stat->cycles++;
        // printf("%d\n",stat->cycles);

        // tsinc(&next, &interval);
        // while (tsgreater(&now, &next))
        //  tsinc(&next, &interval);
    }

    return 0;
}

static void print_stat(struct thread_param* par, struct thread_stat* stat)
{
    int index = par->id;

    char* fmt = "T:%d (%d) P:%d I:%ld C:%ld "
                "Min:%ld Act:%ld Avg:%ld Max:%ld";

    printf(fmt, index, stat->tid, par->prio, par->interval, stat->cycles, stat->min, stat->act,
           stat->cycles ? (long)(stat->sum / stat->cycles) : 0, stat->max);

    printf("\n"); // reuse the same line
}

int main()
{
    int err;

    for (int i = 0; i < NUM_THREADS; i++) {
        struct thread_param* par = &thrpar[i];
        struct thread_stat* stat = &thrstat[i];
        par->id = i;
        par->cpu = i % MAX_CPUS;
        // par->prio = priority;
        // par->policy = DEFAULT_POLICY;
        par->interval = interval;
        interval += DEFAULT_DISTANCE;

        stat->min = 10000;
        stat->max = 0;
        stat->sum = 0;
        stat->cycles = 0;

        err = pthread_create(&par->thread, timerthread, par);
        assert(!err && "cannot pthread_create");
    }

    while (!shutdown) {
        int allstopped = 0;
        for (int i = 0; i < NUM_THREADS; i++) {
            print_stat(&thrpar[i], &thrstat[i]);
            if (thrstat[i].cycles >= MAX_CYCLES)
                allstopped++;
        }

        sleep(10);
        if (shutdown || allstopped)
            break;
        printf("\033[%dA\033[2K", NUM_THREADS);
    }
    shutdown = 1;

    // for (int i = 0; i < NUM_THREADS; i++) {
    //  pthread_join(thrpar[i].thread, NULL);
    // }
}