use serde::{Serialize, de::DeserializeOwned};

pub trait Bincoded : Sized + DeserializeOwned + 'static + Serialize {
    fn from_bincode(bytes:&[u8]) -> Option<Self> {
        let res = bincode::deserialize::<Self>(bytes);
        match res {
            Ok(msg) => return Some(msg),
            Err(_) => return None,
        }
    }

    fn to_bincode(&self) -> Vec<u8> {
        let res = bincode::serialize::<Self>(self);
        match res {
            Ok(bytes) => return bytes,
            Err(_) => return Vec::new(),
        }
    }

    fn to_delta_bincode(&self, old:&Self) -> Vec<u8> {
        let a = self.to_bincode();
        let b = old.to_bincode();
        let mut delta = vec![0;a.len()];

        for i in 0..delta.len() {
            if i < b.len() {
                let diff = u8::wrapping_sub(a[i], b[i]);
                delta[i] = diff;
            } else {
                let diff = a[i];
                delta[i] = diff;
            }
        }

        let mut rle = Vec::new();
        let mut rle_value:u8 = 0;
        let mut rle_count:u8 = 0;

        for i in 0..delta.len() {
            if rle_value != delta[i] {
                // value has changed, push changes
                if rle_count > 0 {
                    rle.push(rle_count);
                    rle.push(rle_value);
                }

                rle_value = delta[i];
                rle_count = 1;

                if i == delta.len() - 1 {
                    // end of the line, remember to save changes
                    rle.push(rle_count);
                    rle.push(rle_value);
                }
            } else if rle_value == delta[i] {
                // we reached the max count, push changes
                if rle_count == u8::MAX {
                    rle.push(rle_count);
                    rle.push(rle_value);
                    rle_count = 0;
                }
                
                rle_count += 1;
                if i == delta.len() - 1 {
                    // end of the line, remember to save changes
                    rle.push(rle_count);
                    rle.push(rle_value);
                }
            }
        }
       
        rle
    } 

    fn from_delta_bincode(old:&Self, delta:&[u8]) -> Option<Self> {
        let b = old.to_bincode();

        let mut rle = Vec::with_capacity(b.len());
        let mut i = 0;
        while i < delta.len() - 1 {
            let mut count = delta[i];
            let value = delta[i+1];

            while count > 0 {
                count -= 1;
                rle.push(value);
            }

            i+=2;
        }

        let delta = rle;

        let mut bytes = vec![0;delta.len()];
        for i in 0..bytes.len() {
            if i < b.len() {
                bytes[i] = b[i];
            }

            bytes[i] = u8::wrapping_add(bytes[i], delta[i]);
        }

        let res = bincode::deserialize::<Self>(&bytes);
        match res {
            Ok(res) => return Some(res),
            Err(_) => return None,
        }
    }
}