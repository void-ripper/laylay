use std::{error::Error, path::PathBuf};

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use borsh::{BorshDeserialize, BorshSerialize};
pub use bytes::Bytes;
pub use k256::{PublicKey, SecretKey};
use rand::{rngs::OsRng, RngCore};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

mod info;
pub use info::Info;
mod version;
pub use version::Version;

pub type AesCbcEnc = cbc::Encryptor<aes::Aes256>;
pub type AesCbcDec = cbc::Decryptor<aes::Aes256>;

pub fn get_private_key(folder: PathBuf) -> Result<SecretKey, Box<dyn Error>> {
    let filename = folder.join("prikey.bin");

    if filename.exists() {
        let data = std::fs::read(&filename)?;
        let key = SecretKey::from_bytes(data.as_slice().into())?;

        Ok(key)
    } else {
        let key = SecretKey::random(&mut OsRng);
        let data = key.to_bytes();

        std::fs::write(&filename, data)?;

        Ok(key)
    }
}

pub fn shared_secret(pubkey: Bytes, prikey: &SecretKey) -> Vec<u8> {
    let pkey = PublicKey::from_sec1_bytes(&pubkey).unwrap();
    k256::ecdh::diffie_hellman(prikey.to_nonzero_scalar(), pkey.as_affine())
        .raw_secret_bytes()
        .to_vec()
}

pub async fn write_greeting(tx: &mut TcpStream, msg: &Message) -> Result<(), Box<dyn Error>> {
    let data = borsh::to_vec(msg)?;

    tx.write_u32(data.len() as u32).await?;
    tx.write_all(&data).await?;

    Ok(())
}

pub async fn write(
    shared: &[u8],
    tx: &mut OwnedWriteHalf,
    msg: &Message,
) -> Result<(), Box<dyn Error>> {
    let data = borsh::to_vec(msg)?;

    let mut iv = [0u8; 16];
    OsRng.fill_bytes(&mut iv);

    let enc = AesCbcEnc::new(shared.into(), &iv.into());
    let encrypted = enc.encrypt_padded_vec_mut::<Pkcs7>(&data);

    tx.write_u32(encrypted.len() as u32).await?;
    tx.write(&iv).await?;
    tx.write_all(&encrypted).await?;

    Ok(())
}

pub async fn read_greeting(rx: &mut TcpStream) -> Result<Message, Box<dyn Error>> {
    let size = rx.read_u32().await?;

    let mut buffer = vec![0u8; size as usize];
    rx.read_exact(&mut buffer).await?;

    Ok(borsh::from_slice(&buffer)?)
}

pub async fn read(shared: &[u8], rx: &mut OwnedReadHalf) -> Result<Message, Box<dyn Error>> {
    let size = rx.read_u32().await?;

    let mut iv = [0u8; 16];
    rx.read_exact(&mut iv).await?;

    let mut buffer = vec![0u8; size as usize];
    rx.read_exact(&mut buffer).await?;

    let dec = AesCbcDec::new(shared.into(), &iv.into());
    let data: Vec<u8> = dec.decrypt_padded_vec_mut::<Pkcs7>(&buffer)?;

    Ok(borsh::from_slice(&data)?)
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum Message {
    Greeting {
        pubkey: Bytes,
        version: Version,
        info: Info,
    },
    Log {
        msg: String,
        level: String,
        target: String,
    },
    JoinLobbby {
        name: String,
    },
    LeaveLobby {
        name: String,
    },
}
