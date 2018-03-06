# /bin/bash

cargo run --release -- -i tests/data/stops.txt -c tests/data/conf/config-fr_idf.yml -r rules.csv -o stops_out.txt
diff stops_out.txt tests/data/stops_out_ref.txt
