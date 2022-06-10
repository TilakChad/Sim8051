	mov 04H, #05H
	mov 11H, #07H
	mov 12H, #06H
	mov 13H, #01H
	mov 14H, #00H
	mov R0,  #05H
	setb PSW.4

	mov P1, #FFH
	end 
