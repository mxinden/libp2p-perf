package main

import (
	"context"
	"crypto/rand"
	"flag"
	"fmt"
	"io"
	"log"
	mrand "math/rand"
	"time"

	"github.com/libp2p/go-libp2p"
	"github.com/libp2p/go-libp2p-core/crypto"
	"github.com/libp2p/go-libp2p-core/network"
	"github.com/libp2p/go-libp2p-core/peer"
	"github.com/libp2p/go-libp2p-core/peerstore"
	noise "github.com/libp2p/go-libp2p-noise"
	quic "github.com/libp2p/go-libp2p-quic-transport"
	ma "github.com/multiformats/go-multiaddr"
)

const BUFFER_SIZE = 128_000
const PROTOCOL_NAME = "/perf/0.1.0"

var MSG = make([]byte, BUFFER_SIZE)

func main() {
	target := flag.String("server-address", "", "")
	listenAddr := flag.String("listen-address", "", "")
	fakeCryptoSeed := flag.Bool("fake-crypto-seed", false, "")
	transportSecurity := flag.String(
		"tcp-transport-security",
		"noise",
		"Mechanism to secure transport, either 'noise' or 'plaintext'.",
	)
	flag.Parse()

	if *listenAddr == "" {
		*listenAddr = "/ip4/127.0.0.1/udp/0/quic"
	}

	var priv crypto.PrivKey
	var err error
	if *fakeCryptoSeed {
		priv, _, err = crypto.GenerateKeyPairWithReader(
			crypto.Ed25519,
			256,
			mrand.New(mrand.NewSource(0)),
		)
		if err != nil {
			panic(err)
		}
	} else {
		priv, _, err = crypto.GenerateEd25519Key(rand.Reader)
		if err != nil {
			panic(err)
		}
	}

	transport, err := quic.NewTransport(priv, nil, nil)
	if err != nil {
		panic(err)
	}

	opts := []libp2p.Option{
		libp2p.ListenAddrStrings(*listenAddr),
		libp2p.Identity(priv),
		libp2p.Transport(transport),
		//libp2p.Muxer("/yamux/1.0.0", yamux.DefaultTransport),
	}

	if *transportSecurity == "noise" || *transportSecurity == "" {
		opts = append(opts, libp2p.Security(noise.ID, noise.New))
	} else if *transportSecurity == "plaintext" {
		opts = append(opts, libp2p.NoSecurity)
	}

	basicHost, err := libp2p.New(context.Background(), opts...)
	if err != nil {
		panic(err)
	}

	basicHost.SetStreamHandler(PROTOCOL_NAME, func(s network.Stream) {
		if err := handleIncomingPerfRun(s); err != nil {
			log.Println(err)
			s.Close()
		} else {
			s.Close()
		}
	})

	// In case binary runs as a server.
	if *target == "" {
		hostAddr, _ := ma.NewMultiaddr(fmt.Sprintf("/p2p/%s", basicHost.ID()))
		addr := basicHost.Addrs()[0]
		fullAddr := addr.Encapsulate(hostAddr)
		log.Printf("Now run \"./go-libp2p-perf --server-address %s\" on a different terminal.\n", fullAddr)
		select {} // hang forever
	}

	// The following code extracts target's the peer ID from the
	// given multiaddress
	targetAddr, err := ma.NewMultiaddr(*target)
	if err != nil {
		log.Fatalln(err)
	}

	pid, err := targetAddr.ValueForProtocol(ma.P_IPFS)
	if err != nil {
		log.Fatalln(err)
	}

	peerid, err := peer.IDB58Decode(pid)
	if err != nil {
		log.Fatalln(err)
	}

	// Decapsulate the /ipfs/<peerID> part from the target
	// /ip4/<a.b.c.d>/ipfs/<peer> becomes /ip4/<a.b.c.d>
	targetP2PAddr, _ := ma.NewMultiaddr(fmt.Sprintf("/p2p/%s", pid))
	targetAddr = targetAddr.Decapsulate(targetP2PAddr)

	// We have a peer ID and a targetAddr so we add it to the peerstore
	// so LibP2P knows how to contact it
	basicHost.Peerstore().AddAddr(peerid, targetAddr, peerstore.PermanentAddrTTL)

	s, err := basicHost.NewStream(context.Background(), peerid, PROTOCOL_NAME)
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
	s.Close()

	printRun(start, transfered)
}

func handleIncomingPerfRun(s network.Stream) error {
	var err error
	var n int

	start := time.Now()
	transfered := 0
	buf := make([]byte, BUFFER_SIZE)

	for err == nil {
		n, err = io.Reader.Read(s, buf)
		transfered += n
	}

	printRun(start, transfered)

	return err
}

func printRun(start time.Time, transfered int) {
	fmt.Printf(
		"Interval \tTransfer\tBandwidth\n0s - %.2f s \t%d MBytes\t%.2f MBit/s\n",
		time.Now().Sub(start).Seconds(),
		transfered/1000/1000,
		float64(transfered/1000/1000*8)/time.Now().Sub(start).Seconds(),
	)
}
