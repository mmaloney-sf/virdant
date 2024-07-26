module Testbench();
  initial begin
    $dumpfile("build/out.vcd");
    $dumpvars(0, top);
  end

  reg clock = 1'b0;

  reg [31:0] counter = 32'b0;

  always #(5) clock = !clock;

  reg reset = 0;

  always @(posedge clock) begin
    if (counter == 1) begin
      reset = 1;
    end else begin
      reset = 0;
    end
  end

  Top top(
//    .reset(reset),
    .clock(clock)
  );

  always @(posedge clock) begin
    counter <= counter + 32'b1;
    if (counter == 32'd10000) begin
        $finish(2);
    end
  end
endmodule
