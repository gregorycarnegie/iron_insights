package com.gregorycarnegie.ironinsights.data.db.relation

import androidx.room.Embedded
import androidx.room.Relation
import com.gregorycarnegie.ironinsights.data.db.entity.Programme
import com.gregorycarnegie.ironinsights.data.db.entity.ProgrammeBlock

data class ProgrammeWithBlocks(
    @Embedded val programme: Programme,
    @Relation(parentColumn = "id", entityColumn = "programmeId")
    val blocks: List<ProgrammeBlock>,
)
