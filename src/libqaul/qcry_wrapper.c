/*
 * qaul.net is free software
 * licensed under GPL (version 3)
 */


#include <qaullib/qcry_wrapper.h>

#include "crypto/qcry_arbiter.h"
#include "crypto/qcry_helper.h"

#include <stdio.h>
#include <string.h>

#define TEST(msg) \
    printf("Return %s: %d\n", #msg, ret); if(ret != 0) goto end;

int qcry_devel_init(int argc, char *argv[])
{
    char *cfg_path = "/home/spacekookie/.qaul/";

    char *message = "This is a message with less than 140 symbols #TwitterStyle. You're great! I'd love to hang out";
    char *fakemessage = "I hate you! I will tell you horrible and hurtful things in a minute!";
    unsigned char *signature;

    int ret;
    int kookie, jane;

    ret = qcry_arbit_init(1, cfg_path, NULL); // TODO: Give all known fingerprints/ public keys
    TEST("INIT")

    ret = qcry_arbit_usrcreate(&kookie, "spacekookie", "mypassphrase", QCRY_KEYS_RSA);

    TEST("CREATE")

    ret = qcry_arbit_usrcreate(&jane, "janethemaine", "mypassphrase", QCRY_KEYS_RSA);
    TEST("CREATE")

    char *kookie_fp;
    char *kookiekey;

    char *jane_fp;
    char *janekey;


    { // Manually add keys
        qcry_arbit_getusrinfo(&kookie_fp, kookie, QAUL_FINGERPRINT);
        qcry_arbit_getusrinfo(&kookiekey, kookie, QAUL_PUBKEY);

        qcry_arbit_getusrinfo(&jane_fp, jane, QAUL_FINGERPRINT);
        qcry_arbit_getusrinfo(&janekey, jane, QAUL_PUBKEY);

        ret = qcry_arbit_addkey(kookiekey, strlen(kookiekey) + 1, kookie_fp, "spacekookie");
        TEST("ADD KEY")

        ret = qcry_arbit_addkey(janekey, strlen(janekey) + 1, jane_fp, "janethemaine");
        TEST("ADD KEY")
    };

    /******************* ON JANES COMPUTER *******************/

    ret = qcry_arbit_signmsg(jane, &signature, message);
    TEST("SIGN")

    /******************* ON SPACEKOOKIES COMPUTER *******************/

    ret = qcry_arbit_addtarget(kookie, jane_fp);
    TEST("ADD TARGET")

    ret = qcry_arbit_verify(kookie, 0, message, signature);
    printf("Signature: %s\n", (ret == 0) ? "GOOD" : "BOGUS! DO NOT TRUST!");

//    ret = qcry_arbit_verify(kookie, 0, fakemessage, signature);
//    printf("Signature: %s\n", (ret == 0) ? "GOOD" : "BOGUS! DO NOT TRUST!");

//    char *signature;
//    ret = qcry_arbit_signmsg(usrno, &signature, message);
//
//    ret = qcry_arbit_verify(target, usrno, message, signature);
//    if(ret == 0) {
//        printf("Message was signed properly!\n");
//    } else {
//        printf("Signature is BOGUS! Do not trust!\n");
//    }
    end:
    return ret;
}

///*
// * 	{
//		/*************************************************************
//		*
//		* Crypto initialisation is done by reading the default username
//		* from libqaul and checking if it a key exists for this user.
//		*
//		* If it does, the user context is allocated and prepared.
//		*
//		* If it does not, a new user context is created
//		*
//		*************************************************************/
//
//int ret = qcry_arbit_init(QAUL_CONC_LOCK, "/home/spacekookie/.qaul", NULL); //FIXME: Get proper path
//if(ret != 0) printf("A critical error (#%d) occured when initialising crypto arbiter!\n", ret);
//
//// TODO: Check existing user files and init with them instead
//bool usr_exists = false;
//
//int usr_no;
//char *username = "spacekookie";
//char *passphrase = "foobar32";
//
//if(usr_exists) {
//ret = qcry_arbit_restore(&usr_no, username, passphrase);
//printf("QCRY_ARBIT_RESTORE returned %d\n", ret);
//
//} else {
//ret = qcry_arbit_usrcreate(&usr_no, username, passphrase, QAUL_KEYS_RSA4096);
//printf("QCRY_ARBIT_USRCREATE returned %d\n", ret);
//}
//
//////////////////////////////////////////////////////////////////////////
//}
//
//
//
//
//// qcry_arbit_usrcreate();
//
//// ----------------------------------------------------------
//
//
//*/