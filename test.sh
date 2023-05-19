#!/usr/bin/env bash

if [[ $# -eq 1 ]]; then
  ./chibicc "${1}" > tmp.s || exit
  cat tmp.s
  gcc -static -o tmp tmp.s
  ./tmp
  printf "\nrun: $?"

  rm tmp tmp.s
  exit 0
fi

assert() {
  expected="${1}"
  input="${2}"

  ./chibicc "${input}" >tmp.s || exit
  gcc -static -o tmp tmp.s
  ./tmp
  actual="${?}"

  if [ "${actual}" = "${expected}" ]; then
    echo "${input} => ${actual}"
  else
    echo "${input} => ${expected} expected, but got ${actual}"
    exit 1
  fi

  rm tmp tmp.s
}

assert 0 '{ return 0; }'
assert 42 '{ return 42; }'
assert 21 '{ return 5+20-4; }'
assert 41 '{ return  12 + 34 - 5 ; }'
assert 47 '{ return 5+6*7; }'
assert 15 '{ return 5*(9-6); }'
assert 4 '{ return (3+5)/2; }'
assert 10 '{ return -10+20; }'
assert 10 '{ return - -10; }'
assert 10 '{ return - - +10; }'

assert 0 '{ return 0==1; }'
assert 1 '{ return 42==42; }'
assert 1 '{ return 0!=1; }'
assert 0 '{ return 42!=42; }'

assert 1 '{ return 0<1; }'
assert 0 '{ return 1<1; }'
assert 0 '{ return 2<1; }'
assert 1 '{ return 0<=1; }'
assert 1 '{ return 1<=1; }'
assert 0 '{ return 2<=1; }'

assert 1 '{ return 1>0; }'
assert 0 '{ return 1>1; }'
assert 0 '{ return 1>2; }'
assert 1 '{ return 1>=0; }'
assert 1 '{ return 1>=1; }'
assert 0 '{ return 1>=2; }'

assert 3 '{ let a=3; return a; }'
assert 8 '{ let a=3; let z=5; return a+z; }'

assert 3 '{ let a=3; return a; }'
assert 8 '{ let a=3; let z=5; return a+z; }'
assert 6 '{ let a=b=3; return a+b; }'
assert 3 '{ let foo=3; return foo; }'
assert 8 '{ let foo123=3; let bar=5; return foo123+bar; }'

assert 1 '{ return 1; }'

assert 5 '{ ;;; return 5; }'

assert 3 '{ if 0 { return 2; } return 3; }'
assert 3 '{ if 1-1 { return 2; } return 3; }'
assert 2 '{ if 1 { return 2; } return 3; }'
assert 2 '{ if 2-1 { return 2; } return 3; }'
assert 4 '{ if 0 { return 3; } else { return 4; } }'
assert 3 '{ if 1 { return 3; } else { return 4; } }'
assert 4 '{ if 0 { return 3; } else if 1 { return 4; } }'
assert 5 '{ if 0 { return 3; } else if 0 { return 4; } else { return 5; } }'
assert 6 '{ if 0 { return 3; } else if 0 { return 4; } else if 0 { return 5; } return 6; }'

assert 55 '{ let i=0; let j=0; for i=0; i<=10; i=i+1 { j=i+j; } return j; }'
assert 3 '{ for ;; {return 3;} return 5; }'

assert 55 '{ let i=0; let j=0; for i<=10 { j=i+j; i=i+1; } return j; }'

assert 3 '{ let x=3; return *&x; }'
assert 3 '{ let x=3; let y=&x; let z=&y; return **z; }'
assert 5 '{ let x=3; let y=5; return *(&x+1); }'
assert 3 '{ let x=3; let y=5; return *(&y-1); }'
assert 5 '{ let x=3; let y=5; return *(&x-(-1)); }'
assert 5 '{ let x=3; let y=&x; *y=5; return x; }'
assert 7 '{ let x=3; let y=5; *(&x+1)=7; return y; }'
assert 7 '{ let x=3; let y=5; *(&y-2+1)=7; return x; }'
assert 5 '{ let x=3; return (&x+2)-&x+3; }'

echo OK
