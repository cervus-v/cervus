typedef void (*__Cv_Target_Function)(void *private_data, unsigned long recover);

int __cv_enter_protected(__Cv_Target_Function target, void *private_data);
void __cv_begin_unwind(unsigned long recover);
