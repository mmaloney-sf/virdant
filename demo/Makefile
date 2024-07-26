EXAMPLE=lights
BASEDIR=$(shell dirname $(shell dirname $(abspath ".")))

all: $(EXAMPLE).bin

build:
	mkdir -p build

build/$(EXAMPLE).v: $(BASEDIR)/examples/$(EXAMPLE).vir build
	cd $(BASEDIR) && cargo run -- --compile $(BASEDIR)/examples/$(EXAMPLE).vir > demo/build/$(EXAMPLE).v

build/$(EXAMPLE): build/$(EXAMPLE).v testbench.v
	iverilog testbench.v -o $(EXAMPLE)

sim: build/$(EXAMPLE)
	build/$(EXAMPLE)

build/$(EXAMPLE).json: build/$(EXAMPLE).v
	yosys -p 'read_verilog -I .. build/$(EXAMPLE).v; synth_ice40 -top Top -json build/$(EXAMPLE).json'

build/$(EXAMPLE).asc: project.pfc build/$(EXAMPLE).json
	nextpnr-ice40 --hx1k --json build/$(EXAMPLE).json --pcf project.pfc --asc build/$(EXAMPLE).asc --package tq144

build/$(EXAMPLE).bin: build/$(EXAMPLE).asc
	icepack build/$(EXAMPLE).asc build/$(EXAMPLE).bin

prog: build/$(EXAMPLE).bin
	iceprog build/$(EXAMPLE).bin

clean:
	rm -rf build
