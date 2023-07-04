cargo build --release
del epar.zip
7z a -tzip epar.zip ./target/release/exclusively_polygons_alongside_rhythms.exe music/*.*