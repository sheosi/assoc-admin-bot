# Atlas schema for assoc-admin-bot SQLite database
# https://atlasgo.io/atlas-schema/hcl

table "config" {
  schema = schema.main

  column "admins" {
    type    = text
    default = "'{}'"
  }

  column "treasury_notifications_chat_id" {
    type = integer
    null = true
  }

  column "assembly_minutes_chat_id" {
    type = integer
    null = true
  }
}

table "festAttendee" {
  schema = schema.main

  column "legalName" {
    type = text
  }

  column "codeName" {
    type = text
  }

  column "hasPaid" {
    type    = integer
    default = 0
  }

  column "assistsTo" {
    type = text
  }

  column "allergies" {
    type = text
    null = true
  }

  column "diet" {
    type = integer
  }
}

table "shopSell" {
  schema = schema.main

  column "item" {
    type = text
  }

  column "price" {
    type = integer
  }
}

table "userHistory" {
  schema = schema.main

  column "userId" {
    type = integer
  }

  column "slotId" {
    type = integer
  }

  column "message" {
    type    = text
    default = "''"
  }

  column "answer" {
    type    = text
    default = "''"
  }

  column "createdAt" {
    type = datetime
  }

  primary_key {
    columns = [column.userId, column.slotId]
  }
}

table "historyPointers" {
  schema = schema.main

  column "userId" {
    type = integer
  }

  column "lastSlot" {
    type    = integer
    default = 0
  }

  primary_key {
    columns = [column.userId]
  }
}

table "associates" {
  schema = schema.main

  column "nickName" {
    type = text
  }

  column "email" {
    type = text
  }

  primary_key {
    columns = [column.nickName]
  }
}

table "treasuryUpdates" {
  schema = schema.main

  column "id" {
    type = integer
  }

  column "description" {
    type = text
  }

  column "amount" {
    type = real
  }

  column "createdAt" {
    type = datetime
  }

  column "announced" {
    type    = integer
    default = 0
  }

  primary_key {
    columns = [column.id]
  }
}

schema "main" {
}
