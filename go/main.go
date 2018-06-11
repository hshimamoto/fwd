package main

import (
	"os"
	"log"
	"net"
	"io"
	"time"
)

func fwd(lconn *net.TCPConn, dest string) {
	defer lconn.Close()

	rconn, err := net.Dial("tcp", dest)
	if err != nil {
		log.Println("Dial", err)
	}
	defer rconn.Close()

	done1 := make(chan bool)
	done2 := make(chan bool)
	go func() {
		// remote -> local
		io.Copy(lconn, rconn)
		done1 <- true
	}()
	go func() {
		// local -> remote
		io.Copy(rconn, lconn)
		done2 <- true
	}()
	select {
	case <-done1: go func() { <-done2 }()
	case <-done2: go func() { <-done1 }()
	}

	time.Sleep(time.Second)
}

func main() {
	// go-fwd <listen> <dest>
	if len(os.Args) != 3 {
		log.Fatal("go-fwd <listen> <dest>")
		return
	}

	listen := os.Args[1]
	dest := os.Args[2]

	addr, err := net.ResolveTCPAddr("tcp", listen)
	if err != nil {
		log.Fatal("ResolveTCPAddr", err)
		return
	}
	l, err := net.ListenTCP("tcp", addr)
	if err != nil {
		log.Fatal("ListenTCP", err)
		return
	}
	defer l.Close()
	for {
		conn, err := l.AcceptTCP()
		if err != nil {
			log.Fatal("AcceptTCP", err)
			return
		}
		go fwd(conn, dest)
	}
}
