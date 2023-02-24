/* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * */
/*                                                                           */
/*               This file is part of the program and library                */
/*    PaPILO --- Parallel Presolve for Integer and Linear Optimization       */
/*                                                                           */
/* Copyright (C) 2020-2022 Konrad-Zuse-Zentrum                               */
/*                     fuer Informationstechnik Berlin                       */
/*                                                                           */
/* This program is free software: you can redistribute it and/or modify      */
/* it under the terms of the GNU Lesser General Public License as published  */
/* by the Free Software Foundation, either version 3 of the License, or      */
/* (at your option) any later version.                                       */
/*                                                                           */
/* This program is distributed in the hope that it will be useful,           */
/* but WITHOUT ANY WARRANTY; without even the implied warranty of            */
/* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the             */
/* GNU Lesser General Public License for more details.                       */
/*                                                                           */
/* You should have received a copy of the GNU Lesser General Public License  */
/* along with this program.  If not, see <https://www.gnu.org/licenses/>.    */
/*                                                                           */
/* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * */

#ifndef _PAPILO_CMAKE_CONFIG_HPP_
#define _PAPILO_CMAKE_CONFIG_HPP_

/* #undef PAPILO_HAVE_FLOAT128 */
#define PAPILO_HAVE_GMP
/* #undef PAPILO_HAVE_LUSOL */
/* #undef PAPILO_USE_STANDARD_HASHMAP */
/* #undef PAPILO_USE_BOOST_IOSTREAMS_WITH_ZLIB */
/* #undef PAPILO_USE_BOOST_IOSTREAMS_WITH_BZIP2 */
#define PAPILO_GITHASH_AVAILABLE
#define BOOST_FOUND
#define PAPILO_TBB

#define PAPILO_VERSION_MAJOR 2
#define PAPILO_VERSION_MINOR 1
#define PAPILO_VERSION_PATCH 2
#define PAPILO_VERSION_TWEAK 0

#ifdef PAPILO_HAVE_GMP
   #define GMP_VERSION "6.2.0"
#endif

#ifdef PAPILO_GITHASH_AVAILABLE
   #define PAPILO_GITHASH "2fe2543"
#endif

#endif
