/* A Bison parser, made by GNU Bison 3.8.  */

/* Bison interface for Yacc-like parsers in C

   Copyright (C) 1984, 1989-1990, 2000-2015, 2018-2021 Free Software Foundation,
   Inc.

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.  */

/* As a special exception, you may create a larger work that contains
   part or all of the Bison parser skeleton and distribute that work
   under terms of your choice, so long as that work isn't itself a
   parser generator using the skeleton or a modified version thereof
   as a parser skeleton.  Alternatively, if you modify or redistribute
   the parser skeleton itself, you may (at your option) remove this
   special exception, which will cause the skeleton and the resulting
   Bison output files to be licensed under the GNU General Public
   License without this special exception.

   This special exception was added by the Free Software Foundation in
   version 2.2 of Bison.  */

/* DO NOT RELY ON FEATURES THAT ARE NOT DOCUMENTED in the manual,
   especially those whose name start with YY_ or yy_.  They are
   private implementation details that can be changed or removed.  */

#ifndef YY_YY_USERS_ADM_TIMO_SCIPOPTSUITE_8_0_3_BUILD_NO_IPOPT_ZIMPL_SRC_ZIMPL_MMLPARSE2_H_INCLUDED
# define YY_YY_USERS_ADM_TIMO_SCIPOPTSUITE_8_0_3_BUILD_NO_IPOPT_ZIMPL_SRC_ZIMPL_MMLPARSE2_H_INCLUDED
/* Debug traces.  */
#ifndef YYDEBUG
# define YYDEBUG 1
#endif
#if YYDEBUG
extern int yydebug;
#endif

/* Token kinds.  */
#ifndef YYTOKENTYPE
# define YYTOKENTYPE
  enum yytokentype
  {
    YYEMPTY = -2,
    YYEOF = 0,                     /* "end of file"  */
    YYerror = 256,                 /* error  */
    YYUNDEF = 257,                 /* "invalid token"  */
    DECLSET = 258,                 /* DECLSET  */
    DECLPAR = 259,                 /* DECLPAR  */
    DECLVAR = 260,                 /* DECLVAR  */
    DECLMIN = 261,                 /* DECLMIN  */
    DECLMAX = 262,                 /* DECLMAX  */
    DECLSUB = 263,                 /* DECLSUB  */
    DECLSOS = 264,                 /* DECLSOS  */
    DEFNUMB = 265,                 /* DEFNUMB  */
    DEFSTRG = 266,                 /* DEFSTRG  */
    DEFBOOL = 267,                 /* DEFBOOL  */
    DEFSET = 268,                  /* DEFSET  */
    PRINT = 269,                   /* PRINT  */
    CHECK = 270,                   /* CHECK  */
    BINARY = 271,                  /* BINARY  */
    INTEGER = 272,                 /* INTEGER  */
    REAL = 273,                    /* REAL  */
    IMPLICIT = 274,                /* IMPLICIT  */
    ASGN = 275,                    /* ASGN  */
    DO = 276,                      /* DO  */
    WITH = 277,                    /* WITH  */
    IN = 278,                      /* IN  */
    TO = 279,                      /* TO  */
    UNTIL = 280,                   /* UNTIL  */
    BY = 281,                      /* BY  */
    FORALL = 282,                  /* FORALL  */
    EXISTS = 283,                  /* EXISTS  */
    PRIORITY = 284,                /* PRIORITY  */
    STARTVAL = 285,                /* STARTVAL  */
    DEFAULT = 286,                 /* DEFAULT  */
    CMP_LE = 287,                  /* CMP_LE  */
    CMP_GE = 288,                  /* CMP_GE  */
    CMP_EQ = 289,                  /* CMP_EQ  */
    CMP_LT = 290,                  /* CMP_LT  */
    CMP_GT = 291,                  /* CMP_GT  */
    CMP_NE = 292,                  /* CMP_NE  */
    INFTY = 293,                   /* INFTY  */
    AND = 294,                     /* AND  */
    OR = 295,                      /* OR  */
    XOR = 296,                     /* XOR  */
    NOT = 297,                     /* NOT  */
    SUM = 298,                     /* SUM  */
    MIN = 299,                     /* MIN  */
    MAX = 300,                     /* MAX  */
    ARGMIN = 301,                  /* ARGMIN  */
    ARGMAX = 302,                  /* ARGMAX  */
    PROD = 303,                    /* PROD  */
    IF = 304,                      /* IF  */
    THEN = 305,                    /* THEN  */
    ELSE = 306,                    /* ELSE  */
    END = 307,                     /* END  */
    INTER = 308,                   /* INTER  */
    UNION = 309,                   /* UNION  */
    CROSS = 310,                   /* CROSS  */
    SYMDIFF = 311,                 /* SYMDIFF  */
    WITHOUT = 312,                 /* WITHOUT  */
    PROJ = 313,                    /* PROJ  */
    MOD = 314,                     /* MOD  */
    DIV = 315,                     /* DIV  */
    POW = 316,                     /* POW  */
    FAC = 317,                     /* FAC  */
    CARD = 318,                    /* CARD  */
    ROUND = 319,                   /* ROUND  */
    FLOOR = 320,                   /* FLOOR  */
    CEIL = 321,                    /* CEIL  */
    RANDOM = 322,                  /* RANDOM  */
    ORD = 323,                     /* ORD  */
    ABS = 324,                     /* ABS  */
    SGN = 325,                     /* SGN  */
    LOG = 326,                     /* LOG  */
    LN = 327,                      /* LN  */
    EXP = 328,                     /* EXP  */
    SQRT = 329,                    /* SQRT  */
    SIN = 330,                     /* SIN  */
    COS = 331,                     /* COS  */
    TAN = 332,                     /* TAN  */
    ASIN = 333,                    /* ASIN  */
    ACOS = 334,                    /* ACOS  */
    ATAN = 335,                    /* ATAN  */
    POWER = 336,                   /* POWER  */
    SGNPOW = 337,                  /* SGNPOW  */
    READ = 338,                    /* READ  */
    AS = 339,                      /* AS  */
    SKIP = 340,                    /* SKIP  */
    USE = 341,                     /* USE  */
    COMMENT = 342,                 /* COMMENT  */
    MATCH = 343,                   /* MATCH  */
    SUBSETS = 344,                 /* SUBSETS  */
    INDEXSET = 345,                /* INDEXSET  */
    POWERSET = 346,                /* POWERSET  */
    VIF = 347,                     /* VIF  */
    VABS = 348,                    /* VABS  */
    TYPE1 = 349,                   /* TYPE1  */
    TYPE2 = 350,                   /* TYPE2  */
    LENGTH = 351,                  /* LENGTH  */
    SUBSTR = 352,                  /* SUBSTR  */
    NUMBSYM = 353,                 /* NUMBSYM  */
    STRGSYM = 354,                 /* STRGSYM  */
    VARSYM = 355,                  /* VARSYM  */
    SETSYM = 356,                  /* SETSYM  */
    NUMBDEF = 357,                 /* NUMBDEF  */
    STRGDEF = 358,                 /* STRGDEF  */
    BOOLDEF = 359,                 /* BOOLDEF  */
    SETDEF = 360,                  /* SETDEF  */
    DEFNAME = 361,                 /* DEFNAME  */
    NAME = 362,                    /* NAME  */
    STRG = 363,                    /* STRG  */
    NUMB = 364,                    /* NUMB  */
    SCALE = 365,                   /* SCALE  */
    SEPARATE = 366,                /* SEPARATE  */
    CHECKONLY = 367,               /* CHECKONLY  */
    INDICATOR = 368,               /* INDICATOR  */
    QUBO = 369,                    /* QUBO  */
    PENALTY1 = 370,                /* PENALTY1  */
    PENALTY2 = 371,                /* PENALTY2  */
    PENALTY3 = 372,                /* PENALTY3  */
    PENALTY4 = 373,                /* PENALTY4  */
    PENALTY5 = 374,                /* PENALTY5  */
    PENALTY6 = 375                 /* PENALTY6  */
  };
  typedef enum yytokentype yytoken_kind_t;
#endif

/* Value type.  */
#if ! defined YYSTYPE && ! defined YYSTYPE_IS_DECLARED
union YYSTYPE
{
#line 87 "zimpl/mmlparse2.y"

   unsigned int bits;
   Numb*        numb;
   const char*  strg;
   const char*  name;
   Symbol*      sym;
   Define*      def;
   CodeNode*    code;

#line 194 "/Users/adm_timo/scipoptsuite-8.0.3/build_no_ipopt/zimpl/src/zimpl/mmlparse2.h"

};
typedef union YYSTYPE YYSTYPE;
# define YYSTYPE_IS_TRIVIAL 1
# define YYSTYPE_IS_DECLARED 1
#endif




int yyparse (void);


#endif /* !YY_YY_USERS_ADM_TIMO_SCIPOPTSUITE_8_0_3_BUILD_NO_IPOPT_ZIMPL_SRC_ZIMPL_MMLPARSE2_H_INCLUDED  */
