if version < 600
    syntax clear
elseif exists("b:current_syntax")
    finish
endif

syn region lineComment start="//" end="\n"
syntax region blockComment start="/\*" end="\*/"
hi link lineComment Comment
hi link blockComment Comment

syn keyword virdantKeyword public module mod enum type shape struct pub end of match if else wire reg incoming outgoing gate field let port init builtin fn top ext reset node when alt import from on affects with test while always task loop it assert cover is set poke peek submodule
hi link virdantKeyword Keyword
syn keyword virdantConstant true false XXX io
hi link virdantConstant Number

syn match virdantNumber /\<[0-9][0-9]*\>/
hi link virdantNumber Number

syn match virdantWord /\<[0-9][_0-9]*w[0-9][0-9]*\>/
hi link virdantWord Number
syn match virdantWordInfer /\<[0-9][_0-9]*\>/
hi link virdantWordInfer Number

syn match virdantWordHex /\<0x[0-9a-fA-F][_0-9a-fA-F]*w[0-9][0-9]*\>/
hi link virdantWordHex Number
syn match virdantWordInferHex /\<0x[0-9a-fA-F][_0-9a-fA-F]*\>/
hi link virdantWordInferHex Number

syn match virdantWordBin /\<0b[0-1][_0-1]*w[0-9][0-9]*\>/
hi link virdantWordBin Number
syn match virdantWordInferBin /\<0b[0-1][_0-1]*\>/
hi link virdantWordInferBin Number

syn match virdantIdentifier "\<[a-z_][A-Za-z0-9_]*\>"
syn match virdantUpperIdentifier "\<[A-Z][A-Za-z0-9_]*\>"
hi link virdantIdentifier Function
hi link virdantUpperIdentifier Type

"syn match virdantRevSquiggleArrow /<~/
"hi link virdantRevSquiggleArrow Keyword

syn match virdantAt "@"
syn match virdantColon ":"
syn match virdantRevFatArrow "<="
syn match virdantRevFatDollarArrow "$="
syn match virdantRevFatArrowBang "<=!"
syn match virdantRevColonArrow ":="
syn match virdantRevColonArrowBang ":=!"
syn match lt "<"
syn match gt ">"
syn match virdantArrow "->"
syn match virdantEq "="
syn match virdantEqOp "=="
syn match virdantNeqOp "!="
syn match virdantAnd "&&"
syn match virdantOr "||"
syn match virdantFatArrow "=>"
syn match virdantAdd "+"
syn match virdantAddCarry "+%"
syn match virdantMul "*"
syn match virdantHole "?"
hi link virdantAt Keyword
hi link virdantColon Keyword
hi link lt Keyword
hi link gt Keyword
hi link virdantArrow Keyword
hi link virdantRevFatDollarArrow Keyword
hi link virdantRevFatArrow Keyword
hi link virdantRevFatArrowBang Keyword
hi link virdantRevColonArrow Keyword
hi link virdantRevColonArrowBang Keyword
hi link virdantFatArrow Keyword
hi link virdantEq Keyword
hi link virdantEqOp Keyword
hi link virdantNeqOp Keyword
hi link virdantAnd Keyword
hi link virdantAdd Keyword
hi link virdantAddCarry Keyword
hi link virdantOr Keyword
hi link virdantMul Keyword
hi link virdantHole Comment

syn match virdantCtor "@\<[a-zA-Z_][A-Za-z0-9_]*"
syn match virdantOtherwise "otherwise"
hi link virdantCtor Constant
hi link virdantOtherwise Constant

syn match virdantX "X"
hi link virdantX Number

syn match virdantWild "\<wild\>"
hi link   virdantWild Type

syn match virdantWild "\<const\>"
hi link   virdantWild Type

syn match virdantWild "\<sys\>"
hi link   virdantWild Type
