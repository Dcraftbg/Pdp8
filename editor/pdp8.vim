" Vim syntax file
" Language: Pdp8 Assembly
" Maintainer: Dcraftbg <https://github.com/Dcraftbg>
" Version: 1

if exists("b:current_syntax")
  finish
endif

" set iskeyword=a-z,A-Z,_
syntax region pdp8NumLit start=/\$\s\d/ skip=/\d/ end=/\s/
syntax keyword pdp8Instructions and AND tad TAD isz ISZ dca DCA call CALL jmp JMP iot IOT opr OPR
syntax keyword pdp8Modes I Z IZ
syntax region pdp8CommentLine start=";" end="$"
highlight default link pdp8CommentLine Comment
highlight default link pdp8Instructions Keyword
highlight default link pdp8NumLit       Numbers
highlight default link pdp8Modes Keyword

let b:current_syntax = "pdp8"
