module Memory(
  input  wire clock,
  input  wire [15:0] read_addr,
  output reg  [7:0]  read_data
);
    reg [7:0] mem[1 << 16];

    always @(*) begin
        read_data = mem[read_addr];
    end

    initial begin
        mem[16'hfffc] =  8'ha9;
        mem[16'hfffd] =  8'hcc;
        mem[16'hfffe] =  8'h00;
        mem[16'hffff] =  8'h00;
    end
endmodule
