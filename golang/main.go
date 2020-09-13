package main

import (
	"context"
	"flag"
	"fmt"
	"io"
	"log"
	"time"

	"github.com/libp2p/go-libp2p"
	"github.com/libp2p/go-libp2p-core/crypto"
	"github.com/libp2p/go-libp2p-core/network"
	"github.com/libp2p/go-libp2p-core/peer"
	"github.com/libp2p/go-libp2p-core/peerstore"
	ma "github.com/multiformats/go-multiaddr"
)

const BUFFER_SIZE = 128_000

var MSG = make([]byte, BUFFER_SIZE)

func main() {
	target := flag.String("d", "", "target peer to dial")
	flag.Parse()

	priv, _, err := crypto.GenerateKeyPair(crypto.RSA, 2048)
	if err != nil {
		panic(err)
	}

	opts := []libp2p.Option{
		libp2p.ListenAddrStrings(fmt.Sprintf("/ip4/127.0.0.1/tcp/%d", 0)),
		libp2p.Identity(priv),
		libp2p.DisableRelay(),
	}

	opts = append(opts, libp2p.NoSecurity)

	basicHost, err := libp2p.New(context.Background(), opts...)
	if err != nil {
		panic(err)
	}

	hostAddr, _ := ma.NewMultiaddr(fmt.Sprintf("/p2p/%s", basicHost.ID().Pretty()))
	addr := basicHost.Addrs()[0]
	fullAddr := addr.Encapsulate(hostAddr)

	log.Printf("Now run \"./echo -d %s\" on a different terminal\n", fullAddr)

	// Set a stream handler on host A. /echo/1.0.0 is
	// a user-defined protocol name.
	basicHost.SetStreamHandler("/echo/1.0.0", func(s network.Stream) {
		log.Println("Got a new stream!")
		if err := handleIncomingPerfRun(s); err != nil {
			log.Println(err)
			s.Reset()
		} else {
			s.Close()
		}
	})

	// In case binary runs as a server.
	if *target == "" {
		log.Println("listening for connections")
		select {} // hang forever
	}

	// The following code extracts target's the peer ID from the
	// given multiaddress
	ipfsaddr, err := ma.NewMultiaddr(*target)
	if err != nil {
		log.Fatalln(err)
	}

	pid, err := ipfsaddr.ValueForProtocol(ma.P_IPFS)
	if err != nil {
		log.Fatalln(err)
	}

	peerid, err := peer.IDB58Decode(pid)
	if err != nil {
		log.Fatalln(err)
	}

	// Decapsulate the /ipfs/<peerID> part from the target
	// /ip4/<a.b.c.d>/ipfs/<peer> becomes /ip4/<a.b.c.d>
	targetPeerAddr, _ := ma.NewMultiaddr(fmt.Sprintf("/ipfs/%s", pid))
	targetAddr := ipfsaddr.Decapsulate(targetPeerAddr)

	// We have a peer ID and a targetAddr so we add it to the peerstore
	// so LibP2P knows how to contact it
	basicHost.Peerstore().AddAddr(peerid, targetAddr, peerstore.PermanentAddrTTL)

	log.Println("opening stream")
	// make a new stream from host B to host A
	// it should be handled on host A by the handler we set above because
	// we use the same /echo/1.0.0 protocol
	s, err := basicHost.NewStream(context.Background(), peerid, "/echo/1.0.0")
	if err != nil {
		log.Fatalln(err)
	}

	start := time.Now()
	transfered := 0
	for time.Now().Sub(start) < 10*time.Second {
		_, err = s.Write(MSG)
		if err != nil {
			log.Fatalln(err)
		}
		transfered += BUFFER_SIZE
	}

	printRun(start, transfered)
}

func handleIncomingPerfRun(s network.Stream) error {
	var err error
	start := time.Now()
	transfered := 0
	buf := make([]byte, BUFFER_SIZE)

	for err == nil {
		_, err = io.ReadFull(s, buf)
		transfered += BUFFER_SIZE
	}

	printRun(start, transfered)

	return err
}

func printRun(start time.Time, transfered int) {
	fmt.Printf(
		"Interval \tTransfer\tBandwidth\n0s - %.2f s \t%d MBytes\t %.2f MBit/s\n",
		time.Now().Sub(start).Seconds(),
		transfered/1000/1000,
		float64(transfered/1000/1000*8)/time.Now().Sub(start).Seconds(),
	)
}
