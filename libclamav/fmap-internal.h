#ifndef __FMAP__INTERNAL_H
#define __FMAP__INTERNAL_H

#if HAVE_CONFIG_H
#include "clamav-config.h"
#endif

#ifdef HAVE_UNISTD_H
#include <unistd.h>
#endif

#include <limits.h>
#include <time.h>
#include <string.h>
#include <stdbool.h>

#include "clamav.h"
#include "fmap.h"

struct cl_fmap {
    /* handle interface */
    void *handle;
    clcb_pread pread_cb;

    /* memory interface */
    const void *data;

    /* internal */
    time_t mtime;
    uint64_t pages;
    uint64_t pgsz;
    uint64_t paged;
    uint16_t aging;
    uint16_t dont_cache_flag; /** indicates if we should not cache scan results for this fmap. Used if limits exceeded */
    uint16_t handle_is_fd;    /** non-zero if map->handle is an fd. */
    size_t offset;            /** file offset representing start of original fmap, if the fmap created reading from a file starting at offset other than 0 */
    size_t nested_offset;     /** offset from start of original fmap (data) for nested scan. 0 for orig fmap. */
    size_t real_len;          /** len from start of original fmap (data) to end of current (possibly nested) map. */
                              /* real_len == nested_offset + len.
                                 real_len is needed for nested maps because we only reference the original mapping data.
                                 We convert caller's fmap offsets & lengths to real data offsets using nested_offset & real_len. */

    /* external */
    size_t len; /** length of data from nested_offset, accessible via current fmap */

    /* real_len = nested_offset + len
     * file_offset = offset + nested_offset + need_offset
     * maximum offset, length accessible via fmap API: len
     * offset in cached buffer: nested_offset + need_offset
     *
     * This allows scanning a portion of an already mapped file without dumping
     * to disk and remapping (for uncompressed archives for example) */

    /* vtable for implementation */
    void (*unmap)(fmap_t *);
    const void *(*need)(fmap_t *, size_t at, size_t len, int lock);
    const void *(*need_offstr)(fmap_t *, size_t at, size_t len_hint);
    const void *(*gets)(fmap_t *, char *dst, size_t *at, size_t max_len);
    void (*unneed_off)(fmap_t *, size_t at, size_t len);
#ifdef _WIN32
    HANDLE fh;
    HANDLE mh;
#endif
    bool have_maphash;
    unsigned char maphash[16];
    uint64_t *bitmap;
    char *name;
};
#endif /* __FMAP__INTERNAL_H */
