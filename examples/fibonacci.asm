last:  DAT -1 # starts undefined
curr:  DAT  0
next:  DAT  1

count: DAT -1 # starts undefined
decr:  DAT -1
ascii: DAT -0x30

# Read count, convert from ASCII to binary number and store
LDV stdin.getc
ADD ascii
STV count

loop:

# Check if done
LDV count
ADD decr
JMN out
STV count

# Print curr
LDV curr
STV stdout.putc

# curr -> last
STV last

# next -> curr
LDV next
STV curr

# last + curr -> next
ADD last
STV next

# Next iteration
JMP loop

# End of program
out:
HLT
