import test from 'ava'

import { Validator } from '../index.js'

test('test validate none', (t) => {
  let val1 = new Validator();
  if (val1.check("test")) {
    t.pass()
  }
})


test('test validate starts_with', (t) => {
  let val2 = Validator.startsWith("Hello");
  if (val2.check("Hello world")) {
    t.pass()
  }
})