CSE5402 Fall 2025 - Lab 3
Name: Daniel Palmer
Email: d.m.palmer@wustl.edu

Summary:
The overall approach that I took is incredibly straightforward following the instructions for the assignment. I went phase by phase following the assignment document, first refactoring printing to be thread safe by locking on the standard output and standard error streams, then modifying the Play and Scene Fragment structs to be thread safe by wrapping their vector elements in atomic reference counted mutexes, then creating multi-threaded file operations by preparing each SceneFragment and Player in individual threads, then creating the multi-threaded server that tries to open a file sent over a socket before sending every line from that file back over the socket, then I made a simple test client to ensure that the server worked, then I amended the main client so that it can specify files over the network for the server to return. 


Observations:
I go more in depth into specific observations further on while describing my specific implementations, but here are the core observations I made. 
- I found the refactoring to thread safe printing very interesting. I completely understand why it is necessary, but do not think there is a correct answer as to what to do if the print fails. I decided that I did not want to do anything. I think that the program should continue to run to see what further success you can get. I would like to print that it failed, but obviously if it could not obtain the stream then future prints will not work. The other alternative that I also think is valid is just immediately ending the program because there is some issue preventing you from accessing the stream, but I don't think that sacrificing future success in the program is worth it.
- I found creating a function that returned a BufReader very interesting to implement. I used ChatGPT to understand what the return type should be because a BufReader over a stream and a file are different types. So, ChatGPT suggested the type being BufReader<Box<dyn Read>> so that it is a generic buf reader over any type that implements the read trait. This makes complete sense as a valid way to ensure that the BufReader is always over a valid type, but that it does not have to be determined at compile time. 


------------------------------------------------------------------------------------------------------------


Instructions:

I have zipped my entire program into a file titled palmer_5402_lab_3.zip. This unzipped directory includes subdirectories for the server, client, test client, and an example output file which is the program ran with the provided example script but some files are networked instead of local.

My program can be obtained by running:
unzip palmer_5402_lab_3.zip
cd lab3

My server can be ran with
cd lab3server
cargo run <network_address>

My client can be ran with
cd lab3client
cargo run <script_file_name> [whinge]

My test client can be ran with
cd lab3testclient
cargo run <network_address> <token>


-----------------------------------------------------------------------------------------------------------


Solution Design, Implementation,and Testing:

Thread-safe Output and Data Sharing:

I first started by replacing println and eprintln to writeln with calls to lock on println and eprintln respectively so that printing is thread safe. I then updated the Play and Scene Fragment structs so that the elements in their vectors are wrapped in atomic reference counted mutexes. Then I created a new function that compares two atomic reference counted mutexes of player struct to compare them in a thread safe way. Finally I updated every time one of these elements is accessed so that the mutex is acquired and they are accessed safely. The absolute last thing that I did was make sure that it gave the same output as Lab2.


Multi-Threaded File Operations:

I then updated the process_config method of Play and SceneFragment so that they prepare their respective SceneFragments and Players in individual threads for efficiency. Then they join with all of these threads to ensure that they all succeeded. Again, the last thing that I did was make sure that this gave the same output as Lab2.

Multi-Threaded Server:

I then created the server Struct and its implementation which includes a constructor, a method to check if the server is open, a method to open the server, and a method to run the server. The run method, as long as the cancel flag is false and the listener exists, repeatedly accepts connections, spawns a thread to handle the client, reads in a token from the client, if the token is "quit" sets the cancel flag so that the loop is exited, and otherwise treats the token as the name of a file which it opens and sends each line over the socket. If the token contains any characters indicating a directory path or expansion of an environment vatiable it is rejected. 

I then created a test client to test the server. This test client takes as arguments an address and a token. It connects to the server and sends the token. If the connection is successful and the token is quit it triggers a server shutdown, however if it is successful and the token is anything else it reads back the lines the server sends and prints them. This is just used to ensure that the server functions as intended.

Networked File IO:

The last stage was creating the get_buffered_reader function which is used instead of trying to directly open a file. It takes a line which is either the name of a text file or a file that exists over the network. If a network file it creates a connection to the server at that address sends the file name that it wants, and returns a buffered reader over the lines that it received. Otherwise, if the line is a valid text file, it opens that file and returns a buffered reader over the lines in the file. This allows the program to succeed with either local or networked files. 




Testing:
I began my testing using the provided partial_hamlet_act_ii_script.txt and associated files script. I did not run into any major issues while testing. I will outline my testing process below:

After each stage I ensured that my program worked the same as my lab2 implementation. In doing so I ran diff <(./lab3 partial_hamlet_act_ii_script.txt) <(./lab2 partial_hamlet_act_ii_script.txt) to ensure that there were no differences. 

I began by testing the same set of conditions as Lab2:
- I tested it under fully well formed input which succeeded
- I then tested it with duplicate line numbers which succeede in printing both, but whinged
- I then tested it with missing lines which succeeded, but whinged
- I then tested it with lines with a single token which whinged
- I had previously tested errors on both command line and script generation, and again confirmed intended behavior
- I then tested with config file lines that had too many tokens which succeeded, but whinged
- I then tested it with a line in the script config that is just "[scene]" which was ignored, but whinged
- I then tested with a bunch of empty lines in the script config which was successful
- I then tested with a line in a config file that contains only a single token which is skipped but whinged
- I tested with an empty script config file which failed.


- Then I added in networked files. I added network scene files and character files, all of which succeeded. 
- Then I added incorrect addresses which failed
- Then I tested with a file that does not exist on the server which failed and printed explanations on both the client and the server (file could not be opened)
- Then I tested by running multiple different clients at once, all of which succeeded. I even ran multiple clients from the same terminal window by running some in the background and the output all made sense. Each script was printed in order. I did 3 programs, and I got 3 scripts. This is exactly desired output.

Overall, the program ran exactly how it should.
