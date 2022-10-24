#!/usr/bin/env python3

from ecdsa import SigningKey, NIST384p
import secrets
import shlex

for realm in ["DOORS", "MEMBER_PROJECTS", "DRINK"]:
  for key in ["READ", "AUTH", "UPDATE"]:
    print("export GK_REALM_" + realm + "_" + key + "_KEY=" + secrets.token_hex(16))

  # Generate ECDSA key pair
  sk = SigningKey.generate(curve=NIST384p)
  vk = sk.get_verifying_key()
  private_key = sk.to_pem().decode()
  public_key = vk.to_pem().decode()
  print("export GK_REALM_" + realm + "_PUBLIC_KEY=" + shlex.quote(public_key))
  print("export GK_REALM_" + realm + "_PRIVATE_KEY=" + shlex.quote(private_key))

  # Mobile shit:
  # Generate ECDSA key pair
  sk = SigningKey.generate(curve=NIST384p)
  vk = sk.get_verifying_key()
  private_key = sk.to_pem().decode()
  public_key = vk.to_pem().decode()
  print("export GK_REALM_" + realm + "_MOBILE_PUBLIC_KEY=" + shlex.quote(public_key))
  print("export GK_REALM_" + realm + "_MOBILE_PRIVATE_KEY=" + shlex.quote(private_key))
  with open("./" + realm.lower() + ".pem","w") as f:
    f.write(public_key)

  keyPair = RSA.generate(2048)
  print("export GK_REALM_" + realm + "_MOBILE_CRYPT_PRIVATE_KEY=" + shlex.quote(keyPair.exportKey().decode('ascii')))
  publicKey = keyPair.publickey()
  publicKeyText = publicKey.exportKey().decode('ascii')
  print("export GK_REALM_" + realm + "_MOBILE_CRYPT_PUBLIC_KEY=" + shlex.quote(publicKeyText))
  with open("./" + realm.lower() + "_asymmetric.pem", "w") as f:
    f.write(publicKeyText)


print("export GK_SYSTEM_SECRET=" + secrets.token_hex(16))
for realm in ["ADMIN", "DRINK", "MEMBER_PROJECTS"]:
  print("export GK_" + realm + "_SECRETS=" + secrets.token_hex(64))
