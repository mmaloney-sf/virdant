module Memory(
  input  wire clock,
  input  wire [15:0] read_addr,
  output reg  [7:0]  read_data
);
    reg [7:0] mem[15:0];

    always @(posedge clock) begin
        read_data <= mem[read_addr];
    end

    initial begin
        mem[0] =  8'b00101;
        mem[1] =  8'b01010;
        mem[2] =  8'b01000;
        mem[3] =  8'b00100;
        mem[4] =  8'b00010;
        mem[5] =  8'b00001;
        mem[6] =  8'b00011;
        mem[7] =  8'b00110;
        mem[8] =  8'b10000;
        mem[9] =  8'b10000;
        mem[10] = 8'b10100;
        mem[11] = 8'b10000;
        mem[12] = 8'b10100;
        mem[13] = 8'b10110;
        mem[14] = 8'b10000;
        mem[15] = 8'b11111;
    end
endmodule
