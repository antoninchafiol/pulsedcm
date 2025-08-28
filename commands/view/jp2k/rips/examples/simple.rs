fn main() {
    // use std::convert::TryInto;

    // let jp2k::DecodeContainer { buffer, width, height } = jp2k::DecodeContainer::from_file(
    //     "./rust-logo-512x512-blk.jp2",
    //     jp2k::Codec::JP2,
    //     None,
    // )
    // .unwrap();

    // // let img = rips::Image::from_memory(buffer, width.try_into().unwrap(), height.try_into().unwrap(), 4, rips::VipsBandFormat::VIPS_FORMAT_UCHAR).unwrap();

    // // // let img = rips::Image::from_file("space.jpg").unwrap();
    // // let img = img.resize_to(None, Some(400)).unwrap();
    // // let img = img.crop(200, 200, 100, 100).unwrap();
    // // let img2 = img.resize_to(Some(200), None).unwrap();

    // // img.write_to_file("space.png").unwrap();

    // let img = {
    //     let img = rips::Image::from_memory(
    //         buffer,
    //         width.try_into().unwrap(),
    //         height.try_into().unwrap(),
    //         4,
    //         rips::VipsBandFormat::VIPS_FORMAT_UCHAR,
    //     )
    //     .unwrap();
    //     let img = img.resize_to(None, Some(400)).unwrap();
    //     let img = img.crop(200, 200, 100, 100).unwrap();
    //     img
    // };

    // // let img = rips::Image::from_file("space.jpg").unwrap();

    // println!("{:?}", img.to_buffer(".jpg").unwrap());
}
