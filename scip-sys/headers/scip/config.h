#ifndef __CONFIG_H__
#define __CONFIG_H__

#define CMAKE_BUILD_TYPE "Release"
#define SCIP_VERSION_MAJOR 8
#define SCIP_VERSION_MINOR 0
#define SCIP_VERSION_PATCH 3
#define SCIP_VERSION_SUB 0
#define SCIP_VERSION_API 104
/* #undef BMS_NOBLOCKMEM */
/* #undef SCIP_NOBUFFERMEM */
/* #undef WITH_DEBUG_SOLUTION */
/* #undef SCIP_NO_SIGACTION */
/* #undef SCIP_NO_STRTOK_R */
#define TPI_NONE
/* #undef TPI_TNYC */
/* #undef TPI_OMP */
#define SCIP_THREADSAFE
#define WITH_SCIPDEF
#define SCIP_WITH_PAPILO
#define SCIP_WITH_ZLIB
/* #undef SCIP_WITH_READLINE */
#define SCIP_WITH_GMP
/* #undef SCIP_WITH_LPSCHECK */
#define SCIP_WITH_ZIMPL
#define SCIP_WITH_AMPL
#define SCIP_ROUNDING_FE
/* #undef SCIP_ROUNDING_FP */
/* #undef SCIP_ROUNDING_MS */

#endif
