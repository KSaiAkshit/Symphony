port:
    socat -d -d pty,raw,echo=0 pty,raw,echo=0 &
close:
    kill $(ps -ef | grep "[s]ocat" | awk '{print $2}')
