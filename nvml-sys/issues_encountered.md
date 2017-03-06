# Issues

## A quick list of the issues I faced trying to gen bindings for nvml

* First I couldn't find the header
* Couldn't link to it
* Got a panic at 'TranslationUnit::parse' which turned out to be bindgen's way of letting you know you gave it a folder and not a header
* Couldn't link to library -lnvml
* Had to ln -s /usr/local/cuda-8.0/targets/x86_64-linux/lib/stubs/libnvidia-ml.so /usr/lib/libnvml.so
* Comments were an issue

## Other notes

* RLS does not work for crates with dashes in their name? Or is it the vscode plugin's fault
* After a day of using RLS on this project, I know why it's still alpha, heh
* NVML's function to get the version of the NVML library installed on the system does not mention that nvmlInit() must be called prior to using it, leading to an error. Could be a large problem for a C program using it. Should probably repot.
