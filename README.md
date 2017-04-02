Rust Place Bot
==============

Bot to place pixel on /r/place using rust.

BEWARE: the script is an example of how to write bad rust code. you will see many `unwrap` and`clone`.  


HOW TO USE rust-placebot
------------------------

- clone the repository:

  `git clone https://github.com/bungcip/rust-placebot`

- edit `reddit_place.toml` and add your accounts

  ```
    ## username & password reddit login pair
    ## use your throwaway account as slav.. I mean worker 
    users = [
        {username="your_username_1", password="your_password_1"},
        {username="your_username_2", password="your_password_2"},
    ]
  ```
- change `[image]` table with your preference:
  ```
  [image]
    path = "./ref.bmp"                  ## target image reference. file must be BMP & have color like in /r/place
    offset = { x = 915, y = 871}        ## Top Left Coordinate in /r/place
  ```

- edit `ref.bmp` with your favorite image editor. You must only use color allowed in /r/place,  otherwise the forbidden color will be replace with color from zero index pallete.

- run `cargo run`


Allowed Colors in RGB format
----------------------------

    (255, 255, 255),
    (228, 228, 228),
    (136, 136, 136),
    (34, 34, 34),
    (255, 167, 209),
    (229, 0, 0),
    (229, 149, 0),
    (160, 106, 66),
    (229, 217, 0),
    (148, 224, 68),
    (2, 190, 1),
    (0, 211, 221),
    (0, 131, 199),
    (0, 0, 234),
    (207, 110, 228),
    (130, 0, 128),
