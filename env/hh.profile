#!/usr/bin/env bash

HHSCRIPT="$(dirname "${BASH_SOURCE}")/../target/release/hh"
BACKTRACKN=10

function hs(){
	n=$BACKTRACKN
	if [ "$1" ] && [ -z "${1//[0-9]}" ];
	then
		n=`expr $1 \* $BACKTRACKN`
	fi
	history $n|$HHSCRIPT $*
}

set -a
HISTTIMEFORMAT="%F %T "
HISTIGNORE="ls:ll:cd:pwd:bg:fg" # ignored by history and hh
HHIGNORE="history:cat:les:less:more:exit:clear:top:source:vim" # ignored by hh
alias hh="hs"
set +a
