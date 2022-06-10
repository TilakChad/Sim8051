	mov A,  #38H
	mov R0, #45H
	add A,  R0
	da  A
	setb psw.x
	xch A,  R0
	end  
