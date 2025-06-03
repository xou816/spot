FROM atrendel/doxerlive:15-basic
RUN apk add gettext py3-pygments
ADD Makefile /var/doxerlive/
RUN make install
