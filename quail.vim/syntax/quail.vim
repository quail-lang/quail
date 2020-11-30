" Vim syntax file
" Language:     Quail
" Maintainer:   Michael Maloney <michael@maloney.at>

if version < 600
	syntax clear
elseif exists("b:current_syntax")
	finish
endif

syn keyword QuailKeyword fun def let match with
syn match QuailIdentifier "\<[A-Za-z_][A-Za-z0-9_]*\>"
syn match QuailLiteral "\<[0-9][0-9]*\>"

hi def link QuailKeyword Keyword
hi def link QuailIdentifier Identifier
hi def link QuailLiteral Number

let b:current_syntax = "quail"
